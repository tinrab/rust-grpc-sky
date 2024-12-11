FROM migrate/migrate:v4.18.1

ENV PASSWORD abc123456
ENV DATABASE mysql://root:$PASSWORD@tcp(mysql:3306)/sky?query

COPY migrations /migrations

ENTRYPOINT ["/usr/bin/env"]
CMD ["sh", "-c", "migrate --path=/migrations --database \"$DATABASE\" up"]
