FROM ubuntu:24.04

RUN useradd -rm -d /home/nb -s /bin/bash -g root -G sudo nb \
    && echo 'nb:nbnb' | chpasswd \
    && mkdir -p /home/nb/.ssh

COPY authorized_keys /home/nb/.ssh/authorized_keys

RUN chown nb /home/nb/.ssh/authorized_keys \
    && chmod 0600 /home/nb/.ssh/authorized_keys \
    && sed -i 's@http://archive@http://cn.archive@g' /etc/apt/sources.list.d/ubuntu.sources \
    && apt update \
    && apt install sudo openssh-server iproute2 -y \
    && sed -ri 's/^#*Port\s*22.*$/Port 2222/' /etc/ssh/sshd_config \
    && sed -ri 's/^[# ]*MaxStartups +[0-9]+.*$/MaxStartups 100/' /etc/ssh/sshd_config \
    && sed -ri 's/^[# ]*MaxSessions +[0-9]+.*$/MaxSessions 30/' /etc/ssh/sshd_config \
    && echo 'nb ALL=NOPASSWD:ALL' >>/etc/sudoers

RUN service ssh start

CMD ["/usr/sbin/sshd","-D"]
