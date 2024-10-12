FROM ubuntu:24.04

RUN useradd -rm -d /home/nb -s /bin/bash -g root -G sudo nb

RUN echo 'nb:nbnb' | chpasswd

RUN mkdir -p /home/nb/.ssh

COPY authorized_keys /home/nb/.ssh/authorized_keys

RUN chown nb /home/nb/.ssh/authorized_keys

RUN chmod 0600 /home/nb/.ssh/authorized_keys

RUN apt update

RUN apt update && apt install sudo openssh-server -y

RUN sed -ri 's/^#*Port\s*22.*$/Port 2222/' /etc/ssh/sshd_config

RUN echo 'nb ALL=NOPASSWD:ALL' >>/etc/sudoers

RUN rm -rf /usr/local/bin

RUN ln -sv /tmp/usr_local_bin /usr/local/bin

RUN service ssh start

CMD ["/usr/sbin/sshd","-D"]
