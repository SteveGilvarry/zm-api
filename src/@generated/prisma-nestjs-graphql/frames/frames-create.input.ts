import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { Frames_Type } from '../prisma/frames-type.enum';
import { Decimal } from '@prisma/client/runtime';
import { GraphQLDecimal } from 'prisma-graphql-type-decimal';
import { transformToDecimal } from 'prisma-graphql-type-decimal';
import { Transform } from 'class-transformer';
import { Type } from 'class-transformer';

@InputType()
export class FramesCreateInput {

    @Field(() => String, {nullable:true})
    Id?: bigint | number;

    @Field(() => String, {nullable:false})
    EventId!: bigint | number;

    @Field(() => Int, {nullable:true})
    FrameId?: number;

    @Field(() => Frames_Type, {nullable:true})
    Type?: keyof typeof Frames_Type;

    @Field(() => Date, {nullable:true})
    TimeStamp?: Date | string;

    @Field(() => GraphQLDecimal, {nullable:true})
    @Type(() => Object)
    @Transform(transformToDecimal)
    Delta?: Decimal;

    @Field(() => Int, {nullable:true})
    Score?: number;
}
