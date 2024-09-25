pub async fn handle_client(client: Socket) {
    let greeting_request = client.recv().await?;
    let greeting_response = response(greeting_request);

    client.send(greeting_response).await?;

    let connection_request = client.recv().await?;

    if connection_request.address.is_domain() {
        let dns_question = dns_socket.send(message).await;
        let dns_answer = dns_socket.recv().await;

        if dns_answer.is_invalid() {
            let connection_response = failure;
            client.send(failure).await?;
            return;
        }
    }

    let destination = connect(connection_request.addr).await?;

    let connection_response = response;
    client.send(response).await?;

    loop {
        let msg = client.recv().await;
        destination.send(msg).await;
        let answer = destination.recv().await;
        client.send(answer).await;
    }
}
