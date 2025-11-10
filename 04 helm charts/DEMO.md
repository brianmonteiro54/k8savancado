# Roteiro Rápido de Demonstração (cola)

```bash
# 1) (opcional) Validar cluster local (Docker Desktop)
./scripts/docker-desktop-setup.sh

# 2) Build imagem
./scripts/build.sh

# 3) Dry-run (render)
helm install --dry-run --debug myrustapp-release ./helm/myrustapp-chart   --namespace dev   --set image.repository=myrustapp   --set image.tag=1.0.0

# 4) Install + port-forward
./scripts/helm-install.sh
# (abre o tunnel; em outra aba:)
curl http://localhost:8080/
# -> confere JSON e "version": "1.0.0"

# 5) Upgrade (nova versão)
IMAGE_TAG=1.0.1 ./scripts/build.sh
IMAGE_TAG=1.0.1 ./scripts/helm-upgrade.sh
curl http://localhost:8080/
# -> confere "version": "1.0.1"

# 6) Rollback (se quiser)
./scripts/helm-rollback.sh

# 7) Uninstall
helm uninstall myrustapp-release -n dev
kubectl delete ns dev --ignore-not-found
```
