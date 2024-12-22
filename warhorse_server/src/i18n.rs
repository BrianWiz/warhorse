use warhorse_protocol::Language;
use crate::config::*;
use crate::error::ServerError;

pub fn hello_message(lang: Language) -> String {
    match lang {
        Language::English => "You are now connected to the Warhorse server".into(),
        Language::Spanish => "Ahora estás conectado al servidor de Warhorse".into(),
        Language::French => "Vous êtes maintenant connecté au serveur Warhorse".into(),
    }
}

pub fn invalid_login(lang: Language) -> ServerError {
    match lang {
        Language::English => "Invalid login, please ensure the information is correct".into(),
        Language::Spanish => "Inicio de sesión inválido, asegúrese de que la información sea correcta".into(),
        Language::French => "Connexion invalide, veuillez vous assurer que les informations sont correctes".into(),
    }
}

pub fn account_name_already_exists(lang: Language) -> ServerError {
    match lang {
        Language::English => "Account name already exists".into(),
        Language::Spanish => "El nombre de la cuenta ya existe".into(),
        Language::French => "Le nom du compte existe déjà".into(),
    }
}

pub fn email_already_exists(lang: Language) -> ServerError {
    match lang {
        Language::English => "Email already exists".into(),
        Language::Spanish => "El correo electrónico ya existe".into(),
        Language::French => "L'email existe déjà".into(),
    }
}

pub fn invalid_email(lang: Language) -> ServerError {
    match lang {
        Language::English => "Invalid email".into(),
        Language::Spanish => "Correo electrónico inválido".into(),
        Language::French => "Email invalide".into(),
    }
}

pub fn invalid_password(lang: Language) -> ServerError {
    match lang {
        Language::English => format!("Passwords must be at least {} characters long", PASSWORD_MIN_LENGTH).into(),
        Language::Spanish => format!("Las contraseñas deben tener al menos {} caracteres", PASSWORD_MIN_LENGTH).into(),
        Language::French => format!("Les mots de passe doivent comporter au moins {} caractères", PASSWORD_MIN_LENGTH).into(),
    }
}

pub fn invalid_account_name(lang: Language) -> ServerError {
    match lang {
        Language::English => format!("Account names must be between {} and {} characters long", ACCOUNT_NAME_MIN_LENGTH, ACCOUNT_NAME_MAX_LENGTH).into(),
        Language::Spanish => format!("Los nombres de cuenta deben tener entre {} y {} caracteres", ACCOUNT_NAME_MIN_LENGTH, ACCOUNT_NAME_MAX_LENGTH).into(),
        Language::French => format!("Les noms de compte doivent comporter entre {} et {} caractères", ACCOUNT_NAME_MIN_LENGTH, ACCOUNT_NAME_MAX_LENGTH).into(),
    }
}

pub fn invalid_display_name(lang: Language) -> ServerError {
    match lang {
        Language::English => format!("Display names must be between {} and {} characters long", DISPLAY_NAME_MIN_LENGTH, DISPLAY_NAME_MAX_LENGTH).into(),
        Language::Spanish => format!("Los nombres de visualización deben tener entre {} y {} caracteres", DISPLAY_NAME_MIN_LENGTH, DISPLAY_NAME_MAX_LENGTH).into(),
        Language::French => format!("Les noms d'affichage doivent comporter entre {} et {} caractères", DISPLAY_NAME_MIN_LENGTH, DISPLAY_NAME_MAX_LENGTH).into(),
    }
}
