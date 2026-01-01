# 通过docker方式启动实例
root_dir=$(cd "$(dirname "$0")"; cd ..; pwd)

# 建议使用6.0-8.4.0版本
#redis_version=8.4.0
redis_version=7.4.7

for port in $(seq 6380 6385); \
do \
   docker run -it -d -p ${port}:${port} -p 1${port}:1${port} \
  --privileged=true -v ${root_dir}/${port}/conf/redis.conf:/usr/local/etc/redis/redis.conf \
  --privileged=true -v ${root_dir}/${port}/data:/data \
  --restart always --name redis-${port} --net redis-net \
  --sysctl net.core.somaxconn=1024 redis:${redis_version} redis-server /usr/local/etc/redis/redis.conf; \
done

echo "redis cluster instances start success"
