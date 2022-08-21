import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { TriggersX10WhereInput } from './triggers-x-10-where.input';
import { Type } from 'class-transformer';
import { TriggersX10OrderByWithRelationInput } from './triggers-x-10-order-by-with-relation.input';
import { TriggersX10WhereUniqueInput } from './triggers-x-10-where-unique.input';
import { Int } from '@nestjs/graphql';
import { TriggersX10ScalarFieldEnum } from './triggers-x-10-scalar-field.enum';

@ArgsType()
export class FindManyTriggersX10Args {

    @Field(() => TriggersX10WhereInput, {nullable:true})
    @Type(() => TriggersX10WhereInput)
    where?: TriggersX10WhereInput;

    @Field(() => [TriggersX10OrderByWithRelationInput], {nullable:true})
    orderBy?: Array<TriggersX10OrderByWithRelationInput>;

    @Field(() => TriggersX10WhereUniqueInput, {nullable:true})
    cursor?: TriggersX10WhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [TriggersX10ScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof TriggersX10ScalarFieldEnum>;
}
