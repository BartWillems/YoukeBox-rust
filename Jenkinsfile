pipeline {
    agent any

    environment {
        REPO_SERVER = 'repo.youkebox.be'
        REPO_PATH   = "/var/vhosts/repo/${BUILD}"
    }

    stages {
        stage('Build') {
            steps {
                sh 'make'
            }
        }

        stage('Package') {
            steps {
                sh "make package --environment-overrides BUILD_NO=${env.BUILD_NUMBER}"
            }
        }

        stage('Deploy') {
            steps {
                sh "scp youkebox-*.rpm root@${REPO_SERVER}:${REPO_PATH}/packages/"
                sh "ssh root@${REPO_SERVER} 'createrepo --update ${REPO_PATH}'"
            }
        }
    }

    post {
        always {
            sh 'make clean'
        }
    }
}