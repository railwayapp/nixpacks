use lunatic::net;

fn main() {
    let sender = net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let receiver = net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let receiver_addr = receiver.local_addr().unwrap();

    sender.connect(receiver_addr).expect("couldn't connect");
    sender
        .send("P1NG".as_bytes())
        .expect("couldn't send message");

    let mut buf = [0; 4];
    let len_in = receiver.recv(&mut buf).unwrap();

    assert_eq!(len_in, 4);
    assert_eq!(buf, "P1NG".as_bytes());

    // wasi stdout
    println!("PING-PONG");
}
