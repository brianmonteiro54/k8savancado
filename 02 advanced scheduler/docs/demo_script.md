# Demo Script (cola)
- Spread: `kubectl -n aula02 get pods -l app=stable -o wide`
- Noisy: `kubectl -n aula02 get pods -l app=noisy -o wide && kubectl get nodes -L workload`
- Preemption: `kubectl -n aula02 describe pod -l app=critical | sed -n '/Events/,$p'`
- Evictions: port-forward + `/alloc?mb=1024&chunks=2` e `kubectl -n aula02 get events --sort-by=.lastTimestamp | tail -n 20`
- Affinity: `kubectl -n aula02 get pods -l tier=data -o wide`, `kubectl -n aula02 get pods -l tier=api -o wide`
