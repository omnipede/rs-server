# rs-server

Rust 로 구현한 로우 레벨 멀티 쓰레드 웹서버

# Run

```shell
$ cargo run
```

# Multithreading

멀티 쓰레딩 처리를 위해서 쓰레드 풀을 사용함.   
일정 개수의 무한 루프 쓰레드를 생성 후, 요청이 들어오면 channel 을 통해 job 을 쓰레드 풀로 전송하는 방식으로 구현함.

# Graceful stop

만약 쓰레드가 요청을 처리중이라면, 그냥 쓰레드를 종료시키는 대신 요청을 전부 처리한 뒤 쓰레드를 멈추는 `Graceful stop` 방식을 사용.  
`Graceful stop` 이 제대로 작동하는지 확인하기 위해서 처음 두 요청만 받고 바로 서버를 종료하도록 함.