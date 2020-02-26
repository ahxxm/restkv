FROM alpine
ADD https://23-242328789-gh.circle-artifacts.com/0/target/x86_64-unknown-linux-musl/release/restkv /restkv
RUN chmod +x /restkv
CMD "/restkv"
