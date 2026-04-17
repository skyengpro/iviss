terraform {
  backend "s3" {
    bucket         = "iviss-terraform-state-577638362880"
    key            = "production/terraform.tfstate"
    region         = "eu-central-1"
    dynamodb_table = "iviss-terraform-lock"
    encrypt        = true
  }
}
