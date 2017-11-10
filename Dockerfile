FROM centos:7
MAINTAINER Bart Willems
LABEL Vendor="CentOS" \
      License=GPLv3

ARG db_username
ARG db_password
ARG db_name

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

USER postgres
RUN postgresql start
RUN psql --command "CREATE USER $db_username WITH SUPERUSER PASSWORD '$db_password';"
RUN createdb -O $db_username $db_name

RUN cargo build --release

# Copy the compiled project to the host machine
COPY --from=build-env /youkebox/target .

