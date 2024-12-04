FROM ubuntu:24.04

ARG UID=1000

RUN userdel -r ubuntu \
    && useradd -m -u $UID -d /home/exp -s /bin/bash -g root -G sudo exp \
    && echo 'exp:expexp' | chpasswd \
    && mkdir -p /home/exp/.ssh

COPY entrypoint.sh /entrypoint.sh

COPY authorized_keys /home/exp/.ssh/authorized_keys

COPY scd /bin/scd

COPY expander-exec /bin/expander-exec

RUN chown exp /home/exp/.ssh/authorized_keys \
    && chmod 0600 /home/exp/.ssh/authorized_keys \
    && chmod +x /bin/scd /bin/expander-exec /entrypoint.sh \
    && sed -i 's@http://archive@http://cn.archive@g' /etc/apt/sources.list.d/ubuntu.sources \
    && apt update \
    && apt install xz-utils sudo libopenmpi-dev openssh-server iproute2 -y \
    && /usr/bin/ssh-keygen -A \
    && sed -ri 's/^#*Port\s*22.*$/Port 2222/' /etc/ssh/sshd_config \
    && sed -ri 's/^[# ]*MaxStartups +[0-9]+.*$/MaxStartups 100/' /etc/ssh/sshd_config \
    && sed -ri 's/^[# ]*MaxSessions +[0-9]+.*$/MaxSessions 30/' /etc/ssh/sshd_config \
    && echo 'exp ALL=NOPASSWD:ALL' >>/etc/sudoers

VOLUME /tmp
VOLUME /usr/local/bin

RUN service ssh start

ENTRYPOINT [ "/entrypoint.sh" ]
