FROM centos:7
MAINTAINER Bart Willems
LABEL Vendor="CentOS" \
      License=GPLv3

RUN yum install -y postgresql   \
        postgresql-contrib  \
        postgresql-devel    \
        postgresql-libs     \
        postgresql-server   \
        epel-release        \
        file sudo gcc       \
        openssl-devel

RUN curl -s https://static.rust-lang.org/rustup.sh | sh -s -- --channel=nightly
RUN yum clean all
RUN rm -rf /var/cache/yum

ADD . /youkebox
WORKDIR /youkebox
RUN echo "DATABASE_URL=postgres://db_user:db_pass@localhost/db_name \
APPLICATION_URL=http://localhost:8000 \
YOUTUBE_API_KEY=123456789" >> .env

EXPOSE 5432

USER postgres

RUN initdb -D '/var/lib/pgsql/data'
RUN echo "host all  all    0.0.0.0/0  md5" >> $HOME/data/pg_hba.conf
RUN echo "listen_addresses='*'" >> $HOME/data/postgresql.conf
RUN postgres -D $HOME/data &
RUN echo "Waiting for postgres to start..."
RUN sleep 30
RUN psql --command "CREATE USER db_user WITH SUPERUSER PASSWORD 'db_pass';"
RUN createdb -O db_user db_name

USER root
RUN cargo build --release

# Copy the compiled project to the host machine
COPY --from=build-env /youkebox/target .

