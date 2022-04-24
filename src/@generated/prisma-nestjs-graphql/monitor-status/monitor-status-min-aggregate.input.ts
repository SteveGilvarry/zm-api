import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class Monitor_StatusMinAggregateInput {

    @Field(() => Boolean, {nullable:true})
    MonitorId?: true;

    @Field(() => Boolean, {nullable:true})
    Status?: true;

    @Field(() => Boolean, {nullable:true})
    CaptureFPS?: true;

    @Field(() => Boolean, {nullable:true})
    AnalysisFPS?: true;

    @Field(() => Boolean, {nullable:true})
    CaptureBandwidth?: true;

    @Field(() => Boolean, {nullable:true})
    DayEventDiskSpace?: true;
}
