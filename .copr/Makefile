srpm:
	dnf install -y git rust cargo rpm-build rpmdevtools
	spectool -g -R sesh.spec
	rpmbuild -bs sesh.spec --define "_sourcedir $(shell pwd)" --define "_srcrpmdir $(outdir)"
