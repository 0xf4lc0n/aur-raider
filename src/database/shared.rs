use crate::models::{
    AdditionalPackageData, BasicPackageData, Comment, PackageData, PackageDependency,
};

pub fn create_package_data() -> PackageData {
    PackageData {
        basic: BasicPackageData {
            name: "Test".into(),
            votes: 100,
            version: "1.2".into(),
            popularity: 6.2,
            maintainer: "Tester".into(),
            description: "Sample description".into(),
            last_updated: "2012".into(),
            path_to_additional_data: "/test".into(),
        },
        additional: AdditionalPackageData {
            license: None,
            keywords: None,
            provides: None,
            confilcts: None,
            submitter: "Tester".into(),
            git_clone_url: "some git url".into(),
            first_submitted: "2011".into(),
        },
        comments: vec![
            Comment {
                header: "Someone wrote at 14:15".into(),
                content: "Cool package".into(),
            },
            Comment {
                header: "Foo wrote at 20:30".into(),
                content: "Not bad".into(),
            },
        ],

        dependencies: vec![PackageDependency {
            group: "abc".into(),
            packages: vec!["aaa".into(), "bbb".into(), "ccc".into()],
        }],
    }
}

pub fn assert_pkg(retreived_pkg: &PackageData, generated_pkg: &PackageData) {
        assert_eq!(retreived_pkg.basic.name, generated_pkg.basic.name);
        assert_eq!(retreived_pkg.comments.len(), generated_pkg.comments.len());
        assert_eq!(
            retreived_pkg.dependencies.len(),
            generated_pkg.dependencies.len()
        );
}
