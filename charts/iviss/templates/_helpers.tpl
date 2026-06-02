{{- define "iviss.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "iviss.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{- define "iviss.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "iviss.labels" -}}
helm.sh/chart: {{ include "iviss.chart" . }}
{{ include "iviss.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/part-of: iviss
{{- end }}

{{- define "iviss.selectorLabels" -}}
app.kubernetes.io/name: {{ include "iviss.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{- define "iviss.backend.fullname" -}}
{{ include "iviss.fullname" . }}-backend
{{- end }}

{{- define "iviss.frontend.fullname" -}}
{{ include "iviss.fullname" . }}-frontend
{{- end }}

{{- define "iviss.postgres.fullname" -}}
iviss-postgres
{{- end }}

{{- define "iviss.imageTag" -}}
{{- .Values.global.imageTag | default .Chart.AppVersion }}
{{- end }}

{{- define "iviss.backend.image" -}}
{{ .Values.backend.image.repository }}:{{ .Values.backend.image.tag | default (include "iviss.imageTag" .) }}
{{- end }}

{{- define "iviss.frontend.image" -}}
{{ .Values.frontend.image.repository }}:{{ .Values.frontend.image.tag | default (include "iviss.imageTag" .) }}
{{- end }}

{{- define "iviss.databaseUrl" -}}
postgres://iviss_user:{{ .Values.databasePassword }}@iviss-postgres-rw:5432/iviss_dev
{{- end }}
