# 获取所有Redis容器ID
containers=$(docker ps -a --filter "name=redis" --format "{{.ID}}")

# 检查容器是否存在
if [ -z "$containers" ]; then
  echo "未找到Redis容器"
  exit 0
fi

# 停止并删除容器
for container in ${containers}; do
  echo "停止并删除redis容器: ${container}";
  docker stop ${container}
  docker rm -f ${container}
done

echo "所有Redis容器已删除"

exit;
# 删除network
network_name=$(docker network ls | grep redis-net | awk '{print $2}')

if [ ${#network_name} -gt 0 ]; then
    docker network rm -f redis-net
    echo "delete redis-net success"
fi
