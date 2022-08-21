import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FramesWhereInput } from './frames-where.input';
import { Type } from 'class-transformer';
import { FramesOrderByWithRelationInput } from './frames-order-by-with-relation.input';
import { FramesWhereUniqueInput } from './frames-where-unique.input';
import { Int } from '@nestjs/graphql';
import { FramesScalarFieldEnum } from './frames-scalar-field.enum';

@ArgsType()
export class FindManyFramesArgs {

    @Field(() => FramesWhereInput, {nullable:true})
    @Type(() => FramesWhereInput)
    where?: FramesWhereInput;

    @Field(() => [FramesOrderByWithRelationInput], {nullable:true})
    @Type(() => FramesOrderByWithRelationInput)
    orderBy?: Array<FramesOrderByWithRelationInput>;

    @Field(() => FramesWhereUniqueInput, {nullable:true})
    @Type(() => FramesWhereUniqueInput)
    cursor?: FramesWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [FramesScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof FramesScalarFieldEnum>;
}
