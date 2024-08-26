mod command_line;

// TODO: I'll need to implement the 'Command Pattern' here.
// TODO: I'll also need to implement a sort of queue that stores all received commands.

// Initializes the listening for cli commands
pub async fn listen_console_commands() {
    tokio::spawn(command_line::handle_input());
}
