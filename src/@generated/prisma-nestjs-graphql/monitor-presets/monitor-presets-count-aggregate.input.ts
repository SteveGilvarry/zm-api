import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class MonitorPresetsCountAggregateInput {

    @Field(() => Boolean, {nullable:true})
    Id?: true;

    @Field(() => Boolean, {nullable:true})
    Name?: true;

    @Field(() => Boolean, {nullable:true})
    Type?: true;

    @Field(() => Boolean, {nullable:true})
    Device?: true;

    @Field(() => Boolean, {nullable:true})
    Channel?: true;

    @Field(() => Boolean, {nullable:true})
    Format?: true;

    @Field(() => Boolean, {nullable:true})
    Protocol?: true;

    @Field(() => Boolean, {nullable:true})
    Method?: true;

    @Field(() => Boolean, {nullable:true})
    Host?: true;

    @Field(() => Boolean, {nullable:true})
    Port?: true;

    @Field(() => Boolean, {nullable:true})
    Path?: true;

    @Field(() => Boolean, {nullable:true})
    SubPath?: true;

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
    ControlId?: true;

    @Field(() => Boolean, {nullable:true})
    ControlDevice?: true;

    @Field(() => Boolean, {nullable:true})
    ControlAddress?: true;

    @Field(() => Boolean, {nullable:true})
    DefaultRate?: true;

    @Field(() => Boolean, {nullable:true})
    DefaultScale?: true;

    @Field(() => Boolean, {nullable:true})
    _all?: true;
}
