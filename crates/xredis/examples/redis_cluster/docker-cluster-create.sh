# 获取宿主机ip地址
host_ip=$(ifconfig | grep "inet " | grep -v 127.0.0.1 | awk '{print $2}')

container_name=redis-6380

# 将redis 实例关联起来
# redis --cluster-replicas 是 Redis 集群配置中的关键参数
# 用于指定每个主节点的从节点数量
# --cluster-replicas 1 表示每个主节点配置 1 个从节点
# 这里是3个主节点，3个从节点
# 高可用：建议至少 3 个主节点（--cluster-replicas 1）以实现高可用
# 通常来说，节点数量‌：总节点数 = 主节点数 × (1 + 从节点数)
# 后续通过 redis-cli --cluster reshard 可调整分片和副本配置
docker exec -it redis-6380 bash -c "
cd /usr/local/bin && redis-cli --cluster create ${host_ip}:6380 ${host_ip}:6381 \
    ${host_ip}:6382 ${host_ip}:6383 ${host_ip}:6384 ${host_ip}:6385 \
    --cluster-replicas 1 \
    --cluster-yes
"

echo "redis cluster host_ip:"$host_ip
echo "redis cluster port from 6380~6385"
