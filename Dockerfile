FROM ubuntu
RUN apt-get update
RUN apt-get install -y build-essential g++ cmake wget radare2
RUN wget https://github.com/google/googletest/archive/release-1.8.1.tar.gz
RUN tar xzf release-1.8.1.tar.gz
RUN cd googletest-release-1.8.1/ &&mkdir build && cd build && cmake ..&& make install
