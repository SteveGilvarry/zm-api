import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatesWhereInput } from './states-where.input';
import { Type } from 'class-transformer';
import { StatesOrderByWithRelationInput } from './states-order-by-with-relation.input';
import { StatesWhereUniqueInput } from './states-where-unique.input';
import { Int } from '@nestjs/graphql';
import { StatesScalarFieldEnum } from './states-scalar-field.enum';

@ArgsType()
export class FindManyStatesArgs {

    @Field(() => StatesWhereInput, {nullable:true})
    @Type(() => StatesWhereInput)
    where?: StatesWhereInput;

    @Field(() => [StatesOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<StatesOrderByWithRelationInput>;

    @Field(() => StatesWhereUniqueInput, {nullable:true})
    cursor?: StatesWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [StatesScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof StatesScalarFieldEnum>;
}
