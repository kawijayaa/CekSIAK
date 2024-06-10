FROM debian:latest

WORKDIR /app

RUN apt update && apt install -y
RUN apt install libssl-dev -y

COPY target/ target/
COPY ui.ac.id.pem .
COPY courses.json* .

CMD ["./target/release/ceksiak"]
