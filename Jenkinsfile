pipeline {
    agent any

    environment {
        BUILD = 'dev'
    }

    stages {
        stage('Build') {
            steps {
                sh 'make'
            }
        }

        stage('Package') {
            steps {
                sh 'make package'
            }
        }

        stage('Deploy') {
            steps {
                echo 'scp youkebox-*.rpm root@repo.youkebox.be:/var/vhosts/repo/packages/${BUILD}'
                echo 'ssh'
            }
        }
    }
}