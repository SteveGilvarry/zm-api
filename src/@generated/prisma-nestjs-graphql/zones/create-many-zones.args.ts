import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonesCreateManyInput } from './zones-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyZonesArgs {

    @Field(() => [ZonesCreateManyInput], {nullable:false})
    @Type(() => ZonesCreateManyInput)
    data!: Array<ZonesCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
