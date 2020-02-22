FROM rust:1
ADD https://9-242328789-gh.circle-artifacts.com/0/target/release/restkv /restkv
RUN chmod +x /restkv
CMD "/restkv"