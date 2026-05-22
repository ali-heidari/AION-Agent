package main

import (
    "fmt"
    "net"
    "net/http"
    "strings"
)

func getMachineIP() string {
    conn, err := net.Dial("udp", "8.8.8.8:80")
    if err != nil {
        return "127.0.0.1"
    }
    defer conn.Close()

    localAddr := conn.LocalAddr().(*net.UDPAddr)
    if localAddr == nil || localAddr.IP == nil {
        return "127.0.0.1"
    }
    return localAddr.IP.String()
}

func main() {
    machineIP := getMachineIP()

    http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
        hostHeader := r.Host
        originalIP := strings.Split(hostHeader, ":")[0]
        redirected := originalIP != "" && originalIP != machineIP

        body := fmt.Sprintf("ip=%s redirected=%t original_dest=%s\n", machineIP, redirected, originalIP)
        body += fmt.Sprintf("host_header=%q\n", hostHeader)
        body += "headers:\n"
        for name, values := range r.Header {
            for _, value := range values {
                body += fmt.Sprintf("  %s: %s\n", name, value)
            }
        }

        w.Header().Set("Content-Type", "text/plain")
        w.WriteHeader(http.StatusOK)
        fmt.Fprint(w, body)
    })

    fmt.Printf("Server running on http://0.0.0.0:8123 (machine IP: %s)\n", machineIP)
    http.ListenAndServe(":8123", nil)
}
