export default function Friend(user: { display_name: string; id: string; status: string }) {

  let on_click = (e: Event) => {
    e.preventDefault();
    console.log("Clicked on " + user.display_name);
  }

  return (
    <div>
      <a id={user.id} on:click={on_click} href="#">
        <p><strong>{user.display_name}</strong></p>
        <p>{user.status}</p>
      </a>
    </div>
  );
}
