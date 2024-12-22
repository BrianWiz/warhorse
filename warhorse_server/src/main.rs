use tonic::{transport::Server, Request, Response, Status};
use tokio::sync::{mpsc, RwLock};
use tracing::info;
use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;

type UserId = String;
type ChannelId = String;

use warhorse_protocol::online::{
    online_service_server::{OnlineService, OnlineServiceServer},
    RequestResponse,
    ConnectRequest, ChatMessage, ChannelChatMessageRequest, PrivateChatMessageRequest,
    JoinChannelRequest, LeaveChannelRequest, ChannelList, MyChannelsRequest, ChannelUsersRequest, UserList, User,
};

#[derive(Debug)]
struct OnlineServer {
    connections: Arc<RwLock<HashMap<UserId, Connection>>>,
    channels: Arc<RwLock<HashMap<ChannelId, Vec<UserId>>>>,
}

#[derive(Debug)]
struct Connection {
    tx: mpsc::Sender<Result<ChatMessage, Status>>,
    display_name: String,
}

impl OnlineServer {
    async fn new() -> Self {
        let out = Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
        };

        out.create_channel("general".into()).await;
        out
    }

    async fn create_channel(&self, channel_id: String) {
        let mut channels = self.channels.write().await;
        channels.insert(channel_id, Vec::new());
    }

    async fn register_user(&self, user_id: String, display_name: String, tx: mpsc::Sender<Result<ChatMessage, Status>>) {
        let mut connections = self.connections.write().await;
        connections.insert(user_id, Connection { tx, display_name });
    }

    async fn handle_deregistering_user(&self, user_id: String) {
        let user_id = user_id.clone();
        let connections = self.connections.clone();
        let channels = self.channels.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            let mut connections = connections.write().await;
            let mut channels = channels.write().await;
            connections.remove(&user_id);
            for (channel_id, users) in channels.iter_mut() {
                users.retain(|id| id != &user_id);
                info!("User {} removed from channel {}", user_id, channel_id);
            }
            info!("User disconnected: {}", user_id);
        });
    }
}

#[tonic::async_trait]
impl OnlineService for OnlineServer {
    type StreamMessagesStream = tokio_stream::wrappers::ReceiverStream<Result<ChatMessage, Status>>;

    async fn stream_messages(
        &self,
        request: Request<ConnectRequest>,
    ) -> Result<Response<Self::StreamMessagesStream>, Status> {
        let req = request.into_inner();
        let user_id = req.user_id;
        let display_name = req.display_name;

        info!("New connection from user: {} ({})", display_name, user_id);

        let (tx, rx) = mpsc::channel(32);
        
        self.register_user(user_id.clone(), display_name.clone(), tx).await;
        self.handle_deregistering_user(user_id.clone()).await;

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }

    async fn send_channel_chat_message(
        &self,
        request: Request<ChannelChatMessageRequest>,
    ) -> Result<Response<RequestResponse>, Status> {
        let msg = request.into_inner();
        let connections = self.connections.read().await;
        let channels = self.channels.read().await;

        if let Some(users) = channels.get(&msg.channel_id) {
            let message = ChatMessage {
                from_user_id: msg.from_user_id,
                from_display_name: msg.from_display_name,
                is_private: false,
                channel_id: msg.channel_id,
                content: msg.content,
                timestamp: Utc::now().timestamp(),
            };

            for user_id in users {
                if let Some(connection) = connections.get(user_id) {
                    connection.tx.send(Ok(message.clone())).await.map_err(|_| {
                        Status::internal("Failed to send message")
                    })?;
                }
            }

            Ok(Response::new(RequestResponse {
                success: true,
                error: String::new(),
            }))
        } else {
            Ok(Response::new(RequestResponse {
                success: false,
                error: "Channel not found".into(),
            }))
        }
    }

    async fn send_private_chat_message(
        &self,
        request: Request<PrivateChatMessageRequest>,
    ) -> Result<Response<RequestResponse>, Status> {
        let msg = request.into_inner();
        let connections = self.connections.read().await;
        
        if let Some(sender) = connections.get(&msg.to_user_id) {
            if let Some(from_user) = connections.get(&msg.from_user_id) {
                let message = ChatMessage {
                    from_user_id: msg.from_user_id,
                    from_display_name: from_user.display_name.clone(),
                    is_private: true,
                    channel_id: String::new(),
                    content: msg.content,
                    timestamp: Utc::now().timestamp(),
                };

                sender.tx.send(Ok(message)).await.map_err(|_| {
                    Status::internal("Failed to send message")
                })?;

                Ok(Response::new(RequestResponse {
                    success: true,
                    error: String::new(),
                }))
            } else {
                Ok(Response::new(RequestResponse {
                    success: false,
                    error: "Sender not found".into(),
                }))
            }
        } else {
            Ok(Response::new(RequestResponse {
                success: false,
                error: "Recipient not found".into(),
            }))
        }
    }

    async fn join_channel(
        &self,
        request: Request<JoinChannelRequest>,
    ) -> Result<Response<RequestResponse>, Status> {
        let msg = request.into_inner();
        let mut channels = self.channels.write().await;
        if let Some(users) = channels.get_mut(&msg.channel_id) {
            users.push(msg.user_id.clone());
            info!("User {} joined channel {}", msg.user_id, msg.channel_id);
            Ok(Response::new(RequestResponse {
                success: true,
                error: String::new(),
            }))
        } else {
            Ok(Response::new(RequestResponse {
                success: false,
                error: "Channel not found".into(),
            }))
        }
    }

    async fn leave_channel(
        &self,
        request: Request<LeaveChannelRequest>,
    ) -> Result<Response<RequestResponse>, Status> {
        let msg = request.into_inner();
        let mut channels = self.channels.write().await;
        if let Some(users) = channels.get_mut(&msg.channel_id) {
            users.retain(|id| id != &msg.user_id);
            info!("User {} left channel {}", msg.user_id, msg.channel_id);
            Ok(Response::new(RequestResponse {
                success: true,
                error: String::new(),
            }))
        } else {
            Ok(Response::new(RequestResponse {
                success: false,
                error: "Channel not found".into(),
            }))
        }
    }

    async fn list_channels(
        &self,
        _request: Request<()>,
    ) -> Result<Response<ChannelList>, Status> {
        let channels = self.channels.read().await;
        let mut channel_list = ChannelList { channels: Vec::new() };
        for channel_id in channels.keys() {
            channel_list.channels.push(channel_id.clone());
        }
        Ok(Response::new(channel_list))
    }

    async fn list_my_channels(
        &self,
        request: Request<MyChannelsRequest>,
    ) -> Result<Response<ChannelList>, Status> {
        let msg = request.into_inner();
        let channels = self.channels.read().await;
        let mut channel_list = ChannelList { channels: Vec::new() };
        for (channel_id, users) in channels.iter() {
            if users.contains(&msg.user_id) {
                channel_list.channels.push(channel_id.clone());
            }
        }
        Ok(Response::new(channel_list))
    }

    async fn list_channel_users(
        &self,
        request: Request<ChannelUsersRequest>,
    ) -> Result<Response<UserList>, Status> {
        let msg = request.into_inner();
        let channels = self.channels.read().await;
        if let Some(users) = channels.get(&msg.channel_id) {
            let connections = self.connections.read().await;
            let mut user_list = UserList { users: Vec::new() };
            for user_id in users {
                if let Some(connection) = connections.get(user_id) {
                    user_list.users.push(User {
                        user_id: user_id.clone(),
                        display_name: connection.display_name.clone(),
                    });
                }
            }
            Ok(Response::new(user_list))
        } else {
            Ok(Response::new(UserList { users: Vec::new() }))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    tracing_subscriber::fmt::init();
    
    let addr = "[::1]:50051".parse()?;
    let online_server = OnlineServer::new().await;
    let online_service_server = OnlineServiceServer::new(online_server);

    info!("Chat server listening on {}", addr);

    Server::builder()
        .add_service(online_service_server)
        .serve(addr)
        .await?;

    Ok(())
}
