variable "enable_iex" {
  description = "This is for the iex eventwatch trigger"
  type        = bool
  default     = false
}

variable "enable_fmp" {
  description = "This is for the fmp eventwatch trigger"
  type        = bool
  default     = false
}

variable "enable_py5" {
  description = "This is for the py5 eventwatch trigger"
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