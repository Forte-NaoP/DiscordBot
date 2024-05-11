음성 채널 연결을 관리하는 부분을 connection handler로 분리했을 때
```rust
let manager = songbird::get(ctx)
    .await
    .expect("Songbird Voice client placed in at initialisation.");
```
이부분에서 `future cannot be sent between threads safely` 에러가 발생했다.

찾아보니 
```rust
let voice_states: &HashMap<UserId, VoiceState> = &guild_id
    .to_guild_cached(ctx)
    .unwrap()
    .voice_states;

let user_channel = voice_states
    .get(&command.user.id)
    .and_then(|voice_state| voice_state.channel_id);
```
`to_guild_cached`로 가져오는 `CacheRef`가 `NotSend`였기 때문에 `songbird::get(ctx).await` 호출 시점에도 `voice_states`가 살아있어서 발생한 문제였다.
```rust
let user_channel = {
    // CacheRef is not Send
    let voice_states: &HashMap<UserId, VoiceState> = &guild_id
        .to_guild_cached(ctx)
        .unwrap()
        .voice_states;
    voice_states
        .get(&command.user.id)
        .and_then(|voice_state| voice_state.channel_id)
}; // So must drop here
```
그래서 다음과 같이 블록을 만들어서 `voice_states`를 블록 내에서만 사용하도록 수정했다.