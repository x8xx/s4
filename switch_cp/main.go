package main

import (
    "fmt"
    "switch_cp/infrastructure"
)

func main() {
    fmt.Println("test")
    client := infrastructure.NewClient("127.0.0.1:8888")
    client.Connect()
    buf := make([]byte, 1024)
    buf[0] = 97;
    buf[1] = 100;
    r_buf, err := client.Send(buf)
    if err != nil {
        panic(err.Error())
    }
    fmt.Println(string(r_buf))


    buf[0] = 98;
    buf[1] = 101;
    r_buf, err = client.Send(buf)
    if err != nil {
        panic(err.Error())
    }
    fmt.Println(string(r_buf))

    client.Close();
}
