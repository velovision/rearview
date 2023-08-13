use tiny_http::{Server, Response};

fn main() {
    let server: Server = Server::http("0.0.0.0:8000").unwrap();

    for mut request in server.incoming_requests() {
        println!("received request! method: {:?}, url: {:?}, headers: {:?}",
            request.method(),
            request.url(),
            request.headers()
        );

        let mut response = Response::from_string("hello world");


        // GET request example
        // Test with: curl 192.168.9.1:8000
        if request.method() == &tiny_http::Method::Get {
            // Get url
            let url = request.url();
            if url == "/" {
                println!("Root request");
                response = Response::from_string("hello root");
            } else {
                println!("Non-root request");
                response = Response::from_string("hello non-root");
            }
        }

        // POST request example
        // Test with: curl -X POST -d "jason" 192.168.9.1:8000
        if request.method() == &tiny_http::Method::Post {
            // Get post content
            let mut post_content = String::new();
            request.as_reader().read_to_string(&mut post_content).unwrap();
            println!("POST content: {}", post_content);

            let reply_content = format!("hello {}", post_content);

            response = Response::from_string(reply_content);
        }
        request.respond(response);
    }

}
