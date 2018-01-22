# YoukeBox Makefile
#
# This program is free software; you can redistribute
# it and/or modify it under the terms of the GNU
# General Public License …

SHELL = /bin/sh

srcdir = .

NAME  = youkebox-rust
DESCRIPTION = "YoukeBox backend"
VERSION = 0.1.0
ARCH = x86_64

all: compile

dink:
	echo $(BUILD)

compile:
	cp .env.dist .env
	cp Rocket.toml.dist Rocket.toml
	cargo build --release
	strip ./target/release/youkebox

TMPDIR := $(shell mktemp -d)
TARGET := $(TMPDIR)/opt/YoukeBox/
SYSTEM := $(TMPDIR)/usr/lib/systemd/system/
package:
	mkdir -p $(TARGET)/bin
	mkdir -p $(SYSTEM)

	cp -r ./migrations $(TARGET)
	cp ./target/release/youkebox $(TARGET)/bin
	cp ./build/youkebox-backend.service $(SYSTEM)/youkebox-backend.service
	cp .env $(TARGET)
	cp Rocket.toml $(TARGET)
	
	for PKG in deb rpm; do \
		fpm -s dir -t $$PKG \
			--name $(NAME) \
			--description $(DESCRIPTION) \
			--version $(VERSION) \
			--architecture $(ARCH) \
			--iteration $(BUILD_NO) \
			--depends postgresql-devel \
			--depends openssl-devel \
			--force \
			--after-install build/post_install.sh \
			--config-files /opt/YoukeBox/.env \
			--config-files /opt/YoukeBox/Rocket.toml \
			--chdir $(TMPDIR) \
			.; \
	done
	
	rm -R $(TMPDIR)

clean:
	rm -f youkebox*.deb
	rm -f youkebox*.rpm

.PHONY: clean
