FROM alpine
ADD https://transfer.sh/tJ2jIb/restkv /restkv
RUN chmod +x /restkv
CMD "/restkv"
