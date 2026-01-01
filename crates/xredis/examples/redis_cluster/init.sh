host_ip=$(ifconfig | grep "inet " | grep -v 127.0.0.1 | awk '{print $2}')

root_dir=$(cd "$(dirname "$0")"; cd ..; pwd)


# 创建network，如果存在就删除
network_name=$(docker network ls | grep redis-net | awk '{print $2}')

if [ ${#network_name} -gt 0 ]; then
    docker network rm -f redis-net
fi

docker network create redis-net

# 初始化配置文件redis.conf
for port in $(seq 6380 6385); \
do
  # 先删除原来的目录
  rm -rf ${root_dir}/${port}
  mkdir -p ${root_dir}/${port}/conf  \
  && PORT=${port} HOST_IP=${host_ip} envsubst < ${root_dir}/redis_cluster/redis-cluster.tmpl > ${root_dir}/${port}/conf/redis.conf \
  && mkdir -p ${root_dir}/${port}/data; \
done

echo "init success"
