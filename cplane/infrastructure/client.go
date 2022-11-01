package infrastructure

import (
    "net"
)

type Client struct {
    addr string
    conn net.Conn
}

func NewClient(addr string) *Client {
    return &Client {
        addr: addr,
        conn: nil,
    }
}

func (client *Client) Connect() error {
    conn, err := net.Dial("tcp", client.addr)
    if err != nil {
        return err
    }
    client.conn = conn
    return nil
}


func (client *Client) Close() {
    client.conn.Close()
}


func (client *Client) Send(buf []byte) ([]byte, error) {
    _, err := client.conn.Write(buf)
    if  err != nil {
        return buf, err
    }
    
    r_buf := make([]byte, 1024)
    n, err := client.conn.Read(r_buf)
    if  err != nil {
        return buf, err
    }


    return buf[:n], nil
}
