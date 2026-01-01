root_dir=$(cd "$(dirname "$0")"; cd ..; pwd)

echo "redis cluster init start..."
# 初始化redis 配置
sh $root_dir/redis_cluster/init.sh

# 启动redis 实例
sh $root_dir/redis_cluster/docker-cluster-start.sh

# 执行 redis cluster create操作
sh $root_dir/redis_cluster/docker-cluster-create.sh

echo "redis cluster create success"
