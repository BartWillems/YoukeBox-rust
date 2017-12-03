FROM centos:7
MAINTAINER Bart Willems
LABEL Vendor="CentOS" \
      License=GPLv3

RUN yum install -y postgresql   \
        postgresql-devel    \
        postgresql-libs     \
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

USER root
RUN cargo build --release

# Copy the compiled project to the host machine
COPY --from=build-env /youkebox/target .