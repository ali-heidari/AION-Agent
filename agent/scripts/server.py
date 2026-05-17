from http.server import BaseHTTPRequestHandler, HTTPServer
import socket

def get_machine_ip():
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    try:
        s.connect(('8.8.8.8', 1))
        ip = s.getsockname()[0]
    except Exception:
        ip = '127.0.0.1'
    finally:
        s.close()
    return ip

MACHINE_IP = get_machine_ip()

class IPResponseHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        host_header = self.headers.get('Host', '')
        original_ip = host_header.split(':')[0]
        redirected = bool(original_ip) and original_ip != MACHINE_IP

        if redirected:
            body = f"ip={MACHINE_IP} redirected=true original_dest={original_ip}\n"
        else:
            body = f"ip={MACHINE_IP} redirected=false\n"

        body += f"host_header={host_header!r}\n"
        body += "headers:\n" + "".join(f"  {k}: {v}\n" for k, v in self.headers.items())

        try:
            self.send_response(200)
            self.send_header('Content-Type', 'text/plain')
            self.end_headers()
            self.wfile.write(body.encode('utf-8'))
        except ConnectionError:
            pass

    def log_message(self, format, *args):
        pass

if __name__ == '__main__':
    PORT = 8123
    server = HTTPServer(('0.0.0.0', PORT), IPResponseHandler)
    print(f"Server running on http://0.0.0.0:{PORT} (machine IP: {MACHINE_IP})")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        server.server_close()
