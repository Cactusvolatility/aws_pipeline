variable "enable_ingestion" {
  description = "This is for the lambda eventwatch trigger"
  type        = bool
  default     = false
}

variable "tiingo_api_key" {
  type      = string
  sensitive = true
}

variable "fmp_api_key" {
  type      = string
  sensitive = true
}

variable "tickers" {
  type = list(string)
}

variable "email" {
  type        = list(string)
  description = "subscription email"
}