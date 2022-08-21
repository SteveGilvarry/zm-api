import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MontageLayoutsCreateManyInput } from './montage-layouts-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyMontageLayoutsArgs {

    @Field(() => [MontageLayoutsCreateManyInput], {nullable:false})
    @Type(() => MontageLayoutsCreateManyInput)
    data!: Array<MontageLayoutsCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
