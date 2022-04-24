import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class MonitorPresetsAvgAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;

    @Field(() => Boolean, {nullable:true})
    Format?: true;

    @Field(() => Boolean, {nullable:true})
    Width?: true;

    @Field(() => Boolean, {nullable:true})
    Height?: true;

    @Field(() => Boolean, {nullable:true})
    Palette?: true;

    @Field(() => Boolean, {nullable:true})
    MaxFPS?: true;

    @Field(() => Boolean, {nullable:true})
    Controllable?: true;

    @Field(() => Boolean, {nullable:true})
    DefaultRate?: true;

    @Field(() => Boolean, {nullable:true})
    DefaultScale?: true;
}
