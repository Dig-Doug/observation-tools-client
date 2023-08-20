import React, {useEffect, useMemo, useState} from 'react';
import clsx from 'clsx';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import styles from './index.module.css';
import {
    Md3DRotation,
    MdArrowRight, MdArrowRightAlt,
    MdBuild,
    MdClose,
    MdDeviceHub,
    MdGradient,
    MdHomeRepairService,
    MdRefresh,
    MdReplay,
    MdViewInAr
} from "react-icons/md";
import CodeBlock from '@theme/CodeBlock';

export default function Home() {
    const {siteConfig} = useDocusaurusContext();
    return (<Layout
        description={siteConfig.tagline}>
        <HomepageHeader/>
        <main className={styles.main}>
            <h2>
                How it works
            </h2>
            {STEPS.map((step, stepIndex) => <div key={stepIndex}>
                <StepHeader index={stepIndex} step={STEPS[stepIndex]}/>
                <div className={styles.stepDetail}>
                    {step.image ? <div className={styles.stepImage}
                                       style={{gridRow: `1 / ${STEPS[stepIndex].benefits.length + 1}`}}>{step.image}</div> : <> </>}
                    {STEPS[stepIndex].benefits.map((benefit, benefitIndex) => <React.Fragment
                        key={`${stepIndex}.${benefitIndex}`}>
                        {benefit.image ? <div className={styles.benefitImage}>{benefit.image}</div> : <> </>}
                        <Feature benefit={benefit}/>
                    </React.Fragment>)}
                </div>
            </div>)}
            <div className={styles.getStartedContainer}>
                <div className={styles.getStartedCard}>
                    <h2>
                        Get started
                    </h2>
                    <p>
                        Login to create a free project and start inspecting your data
                    </p>
                    <Link
                        className="button button--primary button--lg"
                        to="https://app.observation.tools/">
                        Login
                    </Link>
                    <Link
                        className="button button--secondary button--lg"
                        to="/docs/quickstart">
                        Read the docs
                    </Link>
                </div>
            </div>
        </main>
    </Layout>);
}

function HomepageHeader() {
    const {siteConfig} = useDocusaurusContext();
    return (<header className={clsx('hero hero--secondary', styles.heroBanner)}>
        <div className="container">
            <h1 className="hero__title">
                Logging meets the <span className={styles.title3d}>3rd dimension</span>
            </h1>
            <h2 className="hero__subtitle">
                {siteConfig.tagline}
            </h2>
            <div className={styles.buttons}>
                <Link
                    className="button button--primary button--lg"
                    to="/docs/quickstart">
                    Get started
                </Link>
            </div>
        </div>
    </header>);
}

function StepHeader({index, step}) {
    return (<div className={styles.stepHeader}>
        <div className={styles.step}>
            <div className={styles.stepLine}>
            </div>
            <div className={styles.stepNumber}>{index + 1}</div>
        </div>
        <h3>
            {step.title}
        </h3>
        <p>
            {step.description}
        </p>
    </div>);
}

function Feature({benefit}) {
    const Icon = benefit.icon;
    return (<div className={styles.feature}>
        <Icon className={styles.featureIcon}/>
        <h4 className={styles.featureTitle}>
            {benefit.title}
        </h4>
        <p className={styles.featureDescription}>
            {benefit.description}
        </p>
    </div>);
}

const CODE = `fn generate_city() {
  let points = generate_city_outline();
  let poly = Polygon2Builder::from_points(points);
  uploader.upload_object2("city_outline", poly);
}
`;

const STEPS = [{
    title: 'Export', description: <>
        Instrument your code with our client libraries to start exporting data from anywhere in your program
    </>, image: <div className={styles.primitivesDemo}>
        <div className={styles.primitivesCode}>
            <CodeBlock
                language="rust"
                showLineNumbers={true}

            >
                {CODE}
            </CodeBlock>
        </div>
        <BrowserWindow className={styles.primitivesBrowserWindow}>
            <svg className={styles.primitivesImg} width="81.865mm" height="48.943mm" version="1.1"
                 viewBox="0 0 81.865 48.943" xmlns="http://www.w3.org/2000/svg">
                <defs>
                    <marker id="DotS" overflow="visible" orient="auto">
                        <path transform="matrix(.2 0 0 .2 1.48 .2)"
                              d="m-2.5-1c0 2.76-2.24 5-5 5s-5-2.24-5-5 2.24-5 5-5 5 2.24 5 5z" fill="context-stroke"
                              fillRule="evenodd" stroke="context-stroke" strokeWidth="1pt"/>
                    </marker>
                </defs>
                <g transform="translate(-56.373 -39.818)">
                    <path
                        d="m60.811 50.863 36.327-7.9284 38.001 14.804-10.405 21.109-28.165-5.8937-25.855 12.684-11.237-14.997z"
                        fill="none" markerMid="url(#DotS)" markerStart="url(#DotS)" stroke="context-stroke"
                        strokeWidth="1.865"/>
                </g>
            </svg>
        </BrowserWindow>
    </div>
    , benefits: [{
        title: 'Toolbox of existing primitives', icon: MdViewInAr, description: (<>
            Model your data with our library of existing primitives and visualize them in seconds
        </>),
    },]
}, {
    title: 'Visualize', description: <>
        Spend time looking at your data, not deciphering text logs or building custom tools
    </>, image: <div className={styles.pixelImages}>
        <PixelImage />
        <MdArrowRightAlt className={styles.pixelArrow}/>
        <PixelImage useColors={true}/>
    </div>,
    benefits: [{
        title: 'Library of data transformations', icon: MdHomeRepairService, description: <>
            Apply transformations to objects to make them easier to understand, e.g. recoloring images, coordinate
            transforms
        </>
    }, {
        title: "Local or remote execution", icon: MdDeviceHub, description: <>
            Data collection is not limited to one machine. Aggregate data from distributed systems/pipelines and
            view it in a single place.
        </>
    }]
}, {
    title: 'Inspect', description: <>
        Quickly find and diagnose issues so you can get back to building
    </>,
    image: <IntersectionWindow/>,
    benefits: [{
        title: "Suite of debug tools", icon: MdBuild, description: <>
            Use our suite of analysis tools to inspect transformations, find intersections between objects, and
            more.
        </>,
    }, {
        title: "Replay state changes", icon: MdReplay, description: <>
            Go through algorithms step-by-step to see where they went wrong.
        </>,
    }]
}];

function BrowserWindow({className, children}) {
    return <>
        <div className={clsx(className, styles.browserContainer)}>
            <div className={styles.browserTopBar}>
                <div className={styles.browserUrl}>
                    https://observation.tools
                    <MdRefresh className={styles.browserRefresh}/>
                </div>
                <MdClose className={styles.browserClose}/>
            </div>
            <div className={styles.browserContent}>
                {children}
            </div>
        </div>
    </>;
}

const INTERSECTION_STATE_COUNT = 3;

function IntersectionWindow() {
    const [index, setIndex] = useState(0);
    useEffect(() => {
        const interval = setInterval(
            () => {
                setIndex((index) => (index + 1) % INTERSECTION_STATE_COUNT);
            },
            2000,
        );
        return () => clearInterval(interval);
    }, [setIndex]);
    return <>
        <BrowserWindow className={styles.intersectionBrowserWindow}>
            <IntersectionWindowImage index={index}/>
            <input type="range" min="0" max={INTERSECTION_STATE_COUNT - 1} value={index} readOnly disabled={true}/>
        </BrowserWindow>
    </>;
}

function IntersectionWindowImage({index}) {
    switch (index) {
        default:
        case 0:
            return <>
                <svg width="80mm" height="50mm" version="1.1" viewBox="0 0 80 50" xmlns="http://www.w3.org/2000/svg"
                     className={styles.intersectionImage}>
                    <g transform="translate(-56.373 -39.818)">
                        <path transform="matrix(.12707 .15144 -.15243 .1279 102.86 46.354)"
                              d="m171.17 115.37-164.1-53.534 128.41-115.35z" fill="context-fill" stroke="context-stroke"
                              strokeWidth="5.042"/>
                        <rect transform="rotate(70)" x="78.265" y="-73.028" width="10.826" height="32.594"
                              fill="context-stroke" stroke="context-stroke"/>
                        <circle cx="99.176" cy="58.714" r="1.6882" fill="#f00" stroke="none"/>
                        <text transform="matrix(0.26458333,0,0,0.26458333,60.770254,40.082271)" fill="#f00"
                              stroke="#f00">
                            <tspan
                                x="147.47266"
                                y="50.434124"
                            >
                                (127,45)
                            </tspan>
                        </text>
                    </g>
                </svg>
            </>;
        case 1:
            return <>
                <svg width="80mm" height="50mm" version="1.1" viewBox="0 0 80 50" xmlns="http://www.w3.org/2000/svg"
                     className={styles.intersectionImage}>
                    <g transform="translate(-56.373 -39.818)">
                        <path transform="matrix(.027513 .19577 -.19705 .027693 114.61 45.793)"
                              d="m171.17 115.37-164.1-53.534 128.41-115.35z" strokeWidth="5.042"/>
                        <rect transform="rotate(70)" x="78.265" y="-73.028" width="10.826" height="32.594"/>
                    </g>
                </svg>
            </>;
        case 2:
            return <>
                <svg width="80mm" height="50mm" version="1.1" viewBox="0 0 80 50" xmlns="http://www.w3.org/2000/svg"
                     className={styles.intersectionImage}>
                    <g transform="translate(-56.373 -39.818)">
                        <path transform="matrix(-.074056 .1833 -.18449 -.07454 124.33 51.022)"
                              d="m171.17 115.37-164.1-53.534 128.41-115.35z" strokeWidth="5.042"/>
                        <rect transform="rotate(70)" x="78.265" y="-73.028" width="10.826" height="32.594"/>
                    </g>
                </svg>
            </>;
    }
}

function PixelImage({ useColors}) {
    const width = 10;
    const height = 10;
    const data = useMemo(() => {
        const data = Array(width * height).fill();
        for (let x = 0; x < width; x++) {
            for (let y = 0; y < height; y++) {
                const index = x + y * width;
                let value = 0;
                if (x > 1 && x < 4) {
                    if (y > 4 && y < 8) {
                        value = 1;
                    }
                }
                const distance = Math.sqrt(Math.pow((x - width), 2) + Math.pow((y - height), 2));
                const radius = width / 2;
                if (distance < radius) {
                    value = 2;
                }
                if (y < x / 2) {
                    value = 3;
                }
                data[index] = value;
            }
        }
        return data;
    }, [width, height]);
    const COLORS = ['var(--ifm-background-surface-color)', 'var(--ifm-color-danger)', 'var(--ifm-color-success)', 'var(--ifm-color-info)'];
    return <div className={styles.pixelImage}
                style={{
                }}
    aria-hidden={true}>
        {data.map((value, i) => {
            return <div className={styles.pixel} key={i} style={{
                backgroundColor: useColors ? COLORS[value] : `var(--ifm-background-surface-color)`,
            }}>
                {useColors ? "" : value}
            </div>;
        })}
    </div>;
}