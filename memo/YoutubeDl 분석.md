실행 순서
YoutubeDl::into()
    YoutubeDl::create_async (Compose trait 구현 사항)
        HttpRequest::create_async (Compose trait 구현 사항)
            HttpRequest::create_stream
                reqwest::Client::get -> reqwest::Response
                reqwest::Response::bytes_stream을 Box<tokio_io::StreamReader> wrap
                Box<tokio_io::StreamReader>를 songbird::HttpStream으로 wrap
            songbird::HttpStream::stream을 AsyncAdapterStream으로 wrap
        songbird::HttpStream을 songbird::AudioStream으로 wrap
    songbird::AudioStream 반환

create_stream에서 resume를 이용해서 계속 파일을 받아오는 것 같음.
연속된 네트워크 스트림에서 특정 부분만 잘라내는건 어려워보이므로 파일로 저장 후 songbird::File을 이용해서 songbird::Input을 생성



