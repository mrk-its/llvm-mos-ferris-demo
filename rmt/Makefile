music.obx: music.rmt music.a65 rmt_feat.a65 rmtplayr.a65 install_xasm
	xasm music.a65

clean:
	rm -f music.obx

install_xasm:
	curl -L https://github.com/pfusik/xasm/releases/download/xasm-3.2.0/xasm_3.2.0-1_amd64.deb --output xasm.deb
	dpkg -i xasm.deb
	touch install_xasm
