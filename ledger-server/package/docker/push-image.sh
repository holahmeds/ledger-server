# This script expects the user to be logged in to the docker registry already.

if [[ $# -eq 0 ]]; then
  echo "No parameters passed. Expected tag suffix"
  exit 1
fi

tag_suffix=$1

tar_file=$(nix-build docker.nix)

image=$(docker image load --input "$tar_file" | cut --delimiter=' ' --fields=3)

docker tag "$image" "ghcr.io/holahmeds/$image-$tag_suffix"
docker push "ghcr.io/holahmeds/$image-$tag_suffix"
