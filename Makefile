NAME=YoukeBox-rust
VERSION=0.1.0
DESCRIPTION="YoukeBox backend"
ARCH=x86_64

all: compile

compile:
	cp env.dist .env
	cp Rocket.toml.dist Rocket.toml
	cargo build --release
	strip ./target/release/youkebox

TMPDIR := $(shell mktemp -d)
TARGET := $(TMPDIR)/opt/YoukeBox/
SYSTEM := $(TMPDIR)/etc/systemd/system/
package: compile
	mkdir -p $(TARGET)/bin
	mkdir -p $(SYSTEM)

	cp ./target/release/youkebox $(TARGET)/bin
	cp ./build/youkebox.service $(SYSTEM)/youkebox.service
	cp .env $(TARGET)
	cp Rocket.toml $(TARGET)
	
	for PKG in deb rpm; do \
		fpm -s dir -t $$PKG \
			--name $(NAME) \
			--description $(DESCRIPTION) \
			--version $(VERSION) \
			--architecture $(ARCH) \
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
