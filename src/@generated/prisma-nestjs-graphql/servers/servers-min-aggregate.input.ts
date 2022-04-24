import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class ServersMinAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;

    @Field(() => Boolean, {nullable:true})
    Protocol?: true;

    @Field(() => Boolean, {nullable:true})
    Hostname?: true;

    @Field(() => Boolean, {nullable:true})
    Port?: true;

    @Field(() => Boolean, {nullable:true})
    PathToIndex?: true;

    @Field(() => Boolean, {nullable:true})
    PathToZMS?: true;

    @Field(() => Boolean, {nullable:true})
    PathToApi?: true;

    @Field(() => Boolean, {nullable:true})
    Name?: true;

    @Field(() => Boolean, {nullable:true})
    State_Id?: true;

    @Field(() => Boolean, {nullable:true})
    Status?: true;

    @Field(() => Boolean, {nullable:true})
    CpuLoad?: true;

    @Field(() => Boolean, {nullable:true})
    TotalMem?: true;

    @Field(() => Boolean, {nullable:true})
    FreeMem?: true;

    @Field(() => Boolean, {nullable:true})
    TotalSwap?: true;

    @Field(() => Boolean, {nullable:true})
    FreeSwap?: true;

    @Field(() => Boolean, {nullable:true})
    zmstats?: true;

    @Field(() => Boolean, {nullable:true})
    zmaudit?: true;

    @Field(() => Boolean, {nullable:true})
    zmtrigger?: true;

    @Field(() => Boolean, {nullable:true})
    zmeventnotification?: true;
}
