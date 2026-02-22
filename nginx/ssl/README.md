Place your TLS certificates here:
- fullchain.pem
- privkey.pem

For development, generate self-signed certificates:

    openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
        -keyout privkey.pem -out fullchain.pem \
        -subj "/CN=localhost"

For production, use Let's Encrypt with certbot or a reverse proxy service.
