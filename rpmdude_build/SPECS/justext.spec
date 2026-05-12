Name:           justext
Version:        1.0
Release:        1%{?dist}
Summary:        Justext is just a text editor
License:        GPLv3

Source0:        justext
Source1:        justext.png
Source2:        justext.desktop

%description
Justext is just a text editor

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}/usr/bin/
mkdir -p %{buildroot}/usr/share/icons/
mkdir -p %{buildroot}/usr/share/applications/
install -m 755 %{SOURCE0} %{buildroot}/usr/bin/
install -m 644 %{SOURCE1} %{buildroot}/usr/share/icons/
install -m 644 %{SOURCE2} %{buildroot}/usr/share/applications/

%files
/usr/bin/justext
/usr/share/icons/justext.png
/usr/share/applications/justext.desktop

%changelog
* Tue May 12 2026 Developer <sunaipa.org> - 1.0
- Initial RPM release
