import React from 'react';
import clsx from 'clsx';
import styles from './styles.module.scss';

const FeatureList = [
    {
        title: 'Instrument your code',
        Svg: require('@site/static/img/undraw_docusaurus_mountain.svg').default,
        description: (
            <>
                Add one of our clients (python, c++, rust, java) to your program and pass it the data you want to visualize
            </>
        ),
    },
    {
        title: 'Run your program to export data',
        Svg: require('@site/static/img/undraw_docusaurus_tree.svg').default,
        description: (
            <>
                While your program runs, the client will transparently export the data to be visualized
            </>
        ),
    },
    {
        title: 'Visualize and debug',
        Svg: require('@site/static/img/undraw_docusaurus_react.svg').default,
        description: (
            <>
                Inspect your program's outputs on our website and use our tools to debug issues
            </>
        ),
    },
];

function Feature({Svg, title, description}) {
    return (
        <div className={clsx('col col--4')}>
            <div className="text--center">
                <Svg className={styles.featureSvg} alt={title}/>
            </div>
            <div className="text--center padding-horiz--md">
                <h3>{title}</h3>
                <p>{description}</p>
            </div>
        </div>
    );
}

export default function HomepageFeatures() {
    return (
        <section className={styles.features}>
            <div className="container">
                <h2>
                    How it works
                </h2>
                <div className="row">
                    {FeatureList.map((props, idx) => (
                        <Feature key={idx} {...props} />
                    ))}
                </div>
            </div>
        </section>
    );
}
