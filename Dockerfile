FROM centos:7
MAINTAINER Bart Willems
LABEL Vendor="CentOS" \
      License=GPLv3

# Rust & YoukeBox dependencies
RUN yum install -y postgresql-devel   \
        epel-release        \
        file sudo gcc       \
        openssl-devel

RUN yum install -y ruby ruby-devel make rpm-build

RUN gem install pfm

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

# Package it to rpm
# Outputs: $name-$version-$build.$arch.rpm
RUN fpm -s dir -t rpm \
    --name "youkebox" \
    --description "YoukeBox backend" \
    --version 0.1.0 \
    --architecture x86_64 \
    target/release/youkebox=/opt/youkebox/bin/youkebox \
    .env_example=/opt/youkebox/.env \
    Rocket.toml=/opt/youkebox/Rocket.toml

# Now this shit needs to be copied to the host machine...
CMD ["/bin/bash"]