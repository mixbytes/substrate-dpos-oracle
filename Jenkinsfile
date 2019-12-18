pipeline {
  agent {
    node {
      label 'rust&&sgx'
    }
  }
  options {
    timeout(time: 1, unit: 'HOURS')
    buildDiscarder(logRotator(numToKeepStr: '14'))
  }
  stages {
    stage('Environment') {
      steps {
        sh './ci/install_rust.sh'
      }
    }
    stage('Build') {
      steps {
        sh 'cargo build --release'
      }
    }
    stage('Archive artifact') {
      steps {
        archiveArtifacts artifacts: 'target/release/substrate-test-node', fingerprint: true
      }
    }
  }
  post {
    changed {
        emailext (
          subject: "Jenkins Build '${env.JOB_NAME} [${env.BUILD_NUMBER}]' is ${currentBuild.currentResult}",
          body: "${env.JOB_NAME} build ${env.BUILD_NUMBER} changed state and is now ${currentBuild.currentResult}\n\nMore info at: ${env.BUILD_URL}",
          to: '${env.RECIPIENTS_SUBSTRATEE}'
        )
    }
  }
}
