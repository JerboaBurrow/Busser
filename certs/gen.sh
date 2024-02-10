openssl req  -nodes -new -x509 -keyout key.pem -out cert.pem -days 365
#openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -sha256 -days 365