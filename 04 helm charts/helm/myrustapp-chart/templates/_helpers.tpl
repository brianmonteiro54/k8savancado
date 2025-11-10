{{- define "myrustapp.name" -}}
{{- .Chart.Name | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "myrustapp.fullname" -}}
{{- if .Release.Name -}}
{{- printf "%s-%s" .Release.Name (include "myrustapp.name" .) | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- include "myrustapp.name" . -}}
{{- end -}}
{{- end -}}

{{- define "myrustapp.labels" -}}
app.kubernetes.io/name: {{ include "myrustapp.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/version: {{ .Chart.AppVersion }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
helm.sh/chart: {{ .Chart.Name }}-{{ .Chart.Version }}
{{- end -}}
